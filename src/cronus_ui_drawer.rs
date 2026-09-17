//! Dedicated Drawer renderer (wave1t geometry parity with the React vaul Drawer).
//!
//! React renders the open drawer portaled to `<body>` with no trigger and no
//! `drawer` wrapper box:
//! `drawer-overlay` (fixed scrim) + `drawer-content` (fixed, bottom-pinned) holding
//! an unslotted handle bar and `drawer-header` > `h2 drawer-title` + `p drawer-description`.
//! The kernel renders the same tree open by default, zero JS. Swipe / overlay-click
//! dismissal needs JS and is not reproduced (React has no close control to mirror).
//! Closed mode (`trigger:"…"` prop, or `open:false` / `defaultOpen:false`; see
//! `cronus_ui_kit::overlay_trigger`): a `button` trigger
//! (`cronus_ui_dialog::trigger_button`) opens `drawer-content` as a native
//! bottom-pinned modal `<dialog>` (scrim on `::backdrop`, vaul's 500ms slide
//! up); Esc and a backdrop click dismiss it. Gap: no swipe.
//! Body (the docs' `mx-auto w-full max-w-sm` column, an unslotted `<div>`):
//! after the header, a `value "350" description:"Calories / day"` item renders
//! the docs' stat block (`font-display text-5xl` figure + uppercase caption),
//! and `drawer-footer` holds `action` items (primary buttons; React's Submit
//! has no handler, so they are plain buttons) plus the `cancel:"…"` outline
//! button that closes the drawer (`DrawerClose`).
//! Description comes from `description:"…"` (props or item config), else extra `text`.
//! Not interact `dialog("drawer")` native `<dialog>` + `showModal()` + SURF.

use crate::cronus_ui_dialog::trigger_button;
use crate::cronus_ui_kit::{
    attr, attr_nonempty, esc, item, label_of, modal_close_attrs, modal_dialog_open,
    overlay_trigger, widget_id,
};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let title = item(comp, "title")
        .filter(|t| !t.is_empty())
        .map(esc)
        .unwrap_or_else(|| label_of(comp));
    let desc = description(comp, &title);
    let title_id = widget_id(comp, "drawer-title");
    let desc_id = widget_id(comp, "drawer-description");
    let (described_by, desc_html) = if desc.is_empty() {
        (String::new(), String::new())
    } else {
        (
            format!(" aria-describedby=\"{desc_id}\""),
            format!("<p id=\"{desc_id}\" data-slot=\"drawer-description\">{desc}</p>"),
        )
    };
    let header = format!(
        "<div data-slot=\"drawer-header\"><h2 id=\"{title_id}\" data-slot=\"drawer-title\">{title}</h2>{desc_html}</div>"
    );
    let trigger = overlay_trigger(comp, "Open");
    let pop_id = widget_id(comp, "drawer");
    let stat = comp
        .items
        .iter()
        .find(|i| i.item_type == "value" && !i.text.is_empty())
        .map(|v| {
            let caption = v
                .config
                .get("description")
                .filter(|d| !d.trim().is_empty())
                .map(|d| format!("<p>{}</p>", esc(d)))
                .unwrap_or_default();
            format!(
                "<div class=\"stat\"><span>{}</span>{caption}</div>",
                esc(&v.text)
            )
        })
        .unwrap_or_default();
    let mut footer: String = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|i| {
            format!(
                "<button type=\"button\" data-slot=\"button\" data-variant=\"primary\">{}</button>",
                esc(&i.text)
            )
        })
        .collect();
    if let Some(cancel) = attr_nonempty(comp, "cancel") {
        let closes = match &trigger {
            Some(_) => modal_close_attrs(&pop_id),
            None => " disabled".to_string(),
        };
        footer.push_str(&format!(
            "<button type=\"button\" data-slot=\"button\" data-variant=\"outline\"{closes}>{}</button>",
            esc(cancel)
        ));
    }
    let footer = if footer.is_empty() {
        String::new()
    } else {
        format!("<div data-slot=\"drawer-footer\">{footer}</div>")
    };
    // The docs wrap header/body/footer in a `mx-auto w-full max-w-sm` column.
    let body = if stat.is_empty() && footer.is_empty() {
        header
    } else {
        format!("<div>{header}{stat}{footer}</div>")
    };
    let inner = format!("<div aria-hidden=\"true\"></div>{body}");
    match trigger {
        None => format!(
            "<div data-slot=\"drawer-overlay\" aria-hidden=\"true\"></div><div role=\"dialog\" aria-labelledby=\"{title_id}\"{described_by} data-slot=\"drawer-content\">{inner}</div>"
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "drawer-trigger");
            format!(
                "{}{}{inner}</dialog>",
                trigger_button(comp, &trigger_id, &pop_id, &trigger),
                modal_dialog_open(
                    &pop_id,
                    "drawer-content",
                    "dialog",
                    &title_id,
                    &described_by,
                    true
                ),
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
    }

    #[test]
    fn react_dom_overlay_content_header_no_trigger() {
        let mut c = stub("drawer", "Filters");
        c.props
            .insert("description".into(), "Narrow the list.".into());
        let html = render(&c);
        let t = widget_id(&c, "drawer-title");
        let d = widget_id(&c, "drawer-description");
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"drawer-overlay\" aria-hidden=\"true\"></div><div role=\"dialog\" aria-labelledby=\"{t}\" aria-describedby=\"{d}\" data-slot=\"drawer-content\"><div aria-hidden=\"true\"></div><div data-slot=\"drawer-header\"><h2 id=\"{t}\" data-slot=\"drawer-title\">Filters</h2><p id=\"{d}\" data-slot=\"drawer-description\">Narrow the list.</p></div></div>"
            )
        );
        assert!(!html.contains("<button"));
        assert!(!html.contains("data-slot=\"drawer\""));
        reject_interact(&html);
    }

    #[test]
    fn description_from_item_config_or_text() {
        let mut c = stub("drawer", "Filters");
        c.items[0]
            .config
            .insert("description".into(), "From config.".into());
        assert!(render(&c).contains("<p id=\"cui-"));
        assert!(render(&c).contains("data-slot=\"drawer-description\">From config.</p>"));

        let mut t = stub("drawer", "Filters");
        t.items.push(extra("text", "Narrow the list."));
        assert!(render(&t).contains("data-slot=\"drawer-description\">Narrow the list.</p>"));
    }

    #[test]
    fn title_item_wins_and_description_is_optional() {
        let mut c = stub("drawer", "Open");
        c.items.push(extra("title", "Filters"));
        let html = render(&c);
        assert!(html.contains("data-slot=\"drawer-title\">Filters</h2>"));
        assert!(!html.contains("drawer-description"));
        assert!(!html.contains("aria-describedby"));
        reject_interact(&html);
    }

    #[test]
    fn description_is_escaped() {
        let mut c = stub("drawer", "Filters");
        c.props.insert("description".into(), "<b>&</b>".into());
        assert!(render(&c).contains(">&lt;b&gt;&amp;&lt;/b&gt;</p>"));
    }

    #[test]
    fn skips_interact_dialog_surf() {
        let c = stub("drawer", "Menu");
        let html = render(&c);
        reject_interact(&html);
    }

    #[test]
    fn open_false_renders_closed_modal_dialog_behind_a_trigger() {
        let mut c = stub("drawer", "Filters");
        c.props.insert("open".into(), "false".into());
        let html = render(&c);
        let tid = widget_id(&c, "drawer-trigger");
        let pid = widget_id(&c, "drawer");
        assert!(html.starts_with(&format!(
            "<button type=\"button\" id=\"{tid}\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{pid}\" command=\"show-modal\" aria-haspopup=\"dialog\">Open</button><dialog id=\"{pid}\" data-slot=\"drawer-content\" role=\"dialog\" aria-modal=\"true\" aria-labelledby="
        )));
        assert!(html.contains("closedby=\"any\""));
        assert!(!html.contains("drawer-overlay"));
        assert!(!html.contains("popover"));
        assert!(html.contains("data-slot=\"drawer-content\""));
        assert!(html.contains("<div aria-hidden=\"true\"></div>"));
        assert!(html.ends_with("</dialog>"));
    }

    #[test]
    fn docs_drawer_has_stat_block_and_footer() {
        let mut c = stub("drawer", "Move goal");
        c.props.insert("trigger".into(), "Open drawer".into());
        c.props.insert(
            "description".into(),
            "Set your daily activity target.".into(),
        );
        c.props.insert("cancel".into(), "Cancel".into());
        let mut v = extra("value", "350");
        v.config
            .insert("description".into(), "Calories / day".into());
        c.items.push(v);
        c.items.push(extra("action", "Submit"));
        let html = render(&c);
        let pid = widget_id(&c, "drawer");
        assert!(html.contains(&format!(
            "<div aria-hidden=\"true\"></div><div><div data-slot=\"drawer-header\"><h2 id=\"{}\" data-slot=\"drawer-title\">Move goal</h2><p id=\"{}\" data-slot=\"drawer-description\">Set your daily activity target.</p></div><div class=\"stat\"><span>350</span><p>Calories / day</p></div><div data-slot=\"drawer-footer\"><button type=\"button\" data-slot=\"button\" data-variant=\"primary\">Submit</button><button type=\"button\" data-slot=\"button\" data-variant=\"outline\" commandfor=\"{pid}\" command=\"close\">Cancel</button></div></div></dialog>",
            widget_id(&c, "drawer-title"),
            widget_id(&c, "drawer-description")
        )), "{html}");
        // The stat is not a description.
        assert!(!html.contains("drawer-description\">350"));
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"drawer-content\"] > div:not([aria-hidden]) {\n  margin: 0 auto; width: 100%; max-width: 24rem;"));
        assert!(css.contains("[data-slot=\"drawer-content\"] .stat > span {\n  font-family: var(--cronus-font-display, inherit); font-size: 3rem; line-height: 1; font-weight: 600; letter-spacing: -0.025em;"));
        assert!(css.contains("[data-slot=\"drawer-footer\"] {\n  display: flex; flex-direction: column; gap: 0.5rem; margin-top: auto; padding: 1rem;"));
        assert!(css.contains("[data-slot=\"drawer-content\"]:modal {\n  position: fixed; inset-block: auto 0; inset-inline: 0; margin: 6rem 0 0; translate: none;\n  width: auto; padding: 0; border-radius: var(--cronus-radius-xl) var(--cronus-radius-xl) 0 0;\n  max-width: none; max-height: none;\n  animation: cronus-slide-in-bottom 500ms cubic-bezier(0.32, 0.72, 0, 1) both;"));
    }

    #[test]
    fn chrome_closed_mode_is_bottom_pinned_modal_dialog() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("dialog[data-slot=\"drawer-content\"]:not([open]) { display: none; }"));
        assert!(css.contains("[data-slot=\"drawer-content\"]:modal {\n  position: fixed; inset-block: auto 0; inset-inline: 0;"));
        assert!(css.contains("[data-slot=\"drawer-content\"]::backdrop {"));
        assert!(!css.contains("[data-slot=\"drawer-content\"][popover]"));
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("drawer", "Menu"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"drawer-content\""));
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
        let overlay = block("[data-slot=\"drawer-overlay\"]");
        assert!(overlay.contains("position: fixed; inset: 0;"));
        assert!(overlay.contains("color-mix(in srgb, black 50%, transparent)"));
        let content = block("[data-slot=\"drawer-content\"]");
        assert!(content
            .contains("position: fixed; inset-inline-start: 0; inset-inline-end: 0; bottom: 0;"));
        assert!(content.contains("border: 1px solid var(--cronus-border)"));
        assert!(content.contains("var(--cronus-radius-xl) var(--cronus-radius-xl) 0 0"));
        assert!(content.contains("background: var(--cronus-surface-floating)"));
        let handle = block("[data-slot=\"drawer-content\"] > [aria-hidden=\"true\"]");
        assert!(handle.contains("width: 3rem; height: 0.375rem; margin: 1rem auto 0;"));
        let header = block("[data-slot=\"drawer-header\"]");
        assert!(header.contains("display: grid; gap: 0.375rem; padding: 1rem;"));
        // Scoped under the header so it beats `html[data-cronus-theme] h2`
        // (font-weight 400, letter-spacing -0.025em); React's title is 600 / normal.
        let title = block("[data-slot=\"drawer-header\"] > [data-slot=\"drawer-title\"]");
        assert!(title.contains("letter-spacing: normal;"));
        assert!(title.contains("font-size: 1.125rem; line-height: 1.75rem; font-weight: 600;"));
        let desc = block("[data-slot=\"drawer-description\"]");
        assert!(desc.contains("font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(!css.contains("[data-slot=\"drawer\"] > button"));
        assert!(!css.contains("zinc-"));
    }
}
