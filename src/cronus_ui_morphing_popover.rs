//! Dedicated MorphingPopover renderer. DOM mirrors React (the content is NOT
//! portaled — it is absolutely positioned inside the root):
//! `<div data-slot="morphing-popover" data-state>` >
//! `<button data-slot="morphing-popover-trigger">` (label in a `<span>`,
//! `icon:` / `icon-end:` glyphs) + `<div data-slot="morphing-popover-content"
//! role="dialog">` > reveal `<div>` > body.
//!
//! Open mode (default, the audited fixture): static open state, the trigger
//! (inert while open in React) is the native button rendered `disabled`.
//!
//! Closed mode (`open:false`, `trigger:"…"`; see `cronus_ui_kit::overlay_trigger`):
//! the trigger's `popovertarget` opens the content as a native `popover="auto"`
//! centred on the trigger, like React's absolute surface (Esc and outside
//! click dismiss it). The morph is CSS: the surface transitions from the
//! trigger's box (`@starting-style`, `interpolate-size`) on React's 450 ms
//! bounce-0.18 spring while the children reveal 120 ms later (200 ms ease-out).
//! `aria-expanded` / `data-state` stay "closed" (no JS to reflect state).
//!
//! Docs content: `field "…" placeholder:"…" rows:N` renders the feedback
//! form (sr-only label + textarea) with a bare footer holding the close
//! button (`close:true`) and `action` buttons; `item "…" icon:… ` rows render
//! the quick-actions menu (`MorphingPopoverButton`, `separator:true` draws a
//! divider before the row, `tone:secondary` mutes it) — each row hides the
//! popover natively. `width:56|64|72|80|88|96` is the docs `w-*` class.
//! Not interact `popover("morphing-popover")` SURF `<details>` overlay.

use crate::cronus_ui_kit::{
    attr_nonempty, esc, flag, item_icon, label_of, overlay_trigger, truthy, widget_id,
};
use crate::parser::{ComponentItemNode, ComponentNode};

fn cfg<'a>(i: &'a ComponentItemNode, key: &str) -> Option<&'a str> {
    i.config
        .get(key)
        .map(String::as_str)
        .filter(|v| !v.is_empty())
}

/// Footer action button (`variant:` / `size:` / `disabled:` config).
fn action(i: &ComponentItemNode) -> String {
    crate::cronus_ui_glass_card::action_button(i)
}

/// Content of the reveal div: the feedback form, the actions menu, or the
/// plain body text (`text` lines after the label).
fn inner(comp: &ComponentNode, texts: &[String], first: &str, pop_id: Option<&str>) -> String {
    let hide = pop_id
        .map(|id| format!(" popovertarget=\"{id}\" popovertargetaction=\"hide\""))
        .unwrap_or_default();
    let rows: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "item" && !i.text.is_empty())
        .collect();
    let actions: Vec<&ComponentItemNode> = comp
        .items
        .iter()
        .filter(|i| matches!(i.item_type.as_str(), "action" | "button") && !i.text.is_empty())
        .collect();
    let field = comp
        .items
        .iter()
        .find(|i| i.item_type == "field" && !i.text.is_empty());
    let close = flag(comp, "close").then(|| {
        format!(
            "<button type=\"button\" data-slot=\"morphing-popover-close\" aria-label=\"Close\"{hide}>{}</button>",
            crate::cronus_ui_icons::svg_or_empty("x")
        )
    });
    if let Some(field) = field {
        let id = widget_id(comp, "field");
        let rows_n: u32 = cfg(field, "rows")
            .and_then(|r| r.trim().parse().ok())
            .unwrap_or(3);
        let placeholder = cfg(field, "placeholder")
            .map(|p| format!(" placeholder=\"{}\"", esc(p)))
            .unwrap_or_default();
        let footer: String = close
            .iter()
            .cloned()
            .chain(actions.iter().map(|a| action(a)))
            .collect();
        let footer = if footer.is_empty() {
            String::new()
        } else {
            format!("<div data-slot=\"morphing-popover-footer\" class=\"bare\">{footer}</div>")
        };
        return format!(
            "<form><label for=\"{id}\">{}</label><textarea id=\"{id}\" rows=\"{rows_n}\"{placeholder}></textarea>{footer}</form>",
            esc(&field.text)
        );
    }
    if !rows.is_empty() {
        let list: String = rows
            .iter()
            .map(|r| {
                let sep = if cfg(r, "separator").is_some_and(truthy) {
                    "<div></div>"
                } else {
                    ""
                };
                let class = if r.tone.as_deref() == Some("secondary") {
                    " class=\"muted\""
                } else {
                    ""
                };
                format!(
                    "{sep}<button type=\"button\" data-slot=\"morphing-popover-button\"{class}{hide}>{}{}</button>",
                    item_icon(r),
                    esc(&r.text)
                )
            })
            .collect();
        return format!("<div data-slot=\"morphing-popover-body\" class=\"menu\">{list}</div>");
    }
    let body = texts.iter().skip(1).cloned().collect::<Vec<_>>().join("");
    let mut out = if body.is_empty() {
        first.to_string()
    } else {
        body
    };
    if close.is_some() || !actions.is_empty() {
        out.push_str("<div data-slot=\"morphing-popover-footer\">");
        out.push_str(&close.unwrap_or_default());
        for a in &actions {
            out.push_str(&action(a));
        }
        out.push_str("</div>");
    }
    out
}

pub fn render(comp: &ComponentNode) -> String {
    let ts: Vec<String> = comp
        .items
        .iter()
        .filter(|i| {
            matches!(i.item_type.as_str(), "label" | "text" | "title") && !i.text.is_empty()
        })
        .map(|i| esc(&i.text))
        .collect();
    let first = ts.first().cloned().unwrap_or_else(|| label_of(comp));
    let aria = attr_nonempty(comp, "aria-label")
        .map(|v| esc(v))
        .unwrap_or_else(|| "Details".into());
    // Trigger glyphs come from the component props only: an `icon:` after an
    // item line belongs to that row.
    let icon = comp
        .props
        .get("icon")
        .map(|i| crate::cronus_ui_icons::svg_or_empty(i))
        .unwrap_or_default();
    let icon_end = comp
        .props
        .get("icon-end")
        .map(|i| crate::cronus_ui_icons::svg_or_empty(i))
        .unwrap_or_default();
    let width = match attr_nonempty(comp, "width") {
        Some(w @ ("56" | "64" | "72" | "80" | "88" | "96")) => format!(" class=\"w-{w}\""),
        _ => String::new(),
    };
    match overlay_trigger(comp, &first) {
        None => format!(
            "<div data-slot=\"morphing-popover\" data-state=\"open\"><button type=\"button\" data-slot=\"morphing-popover-trigger\" data-state=\"open\" aria-haspopup=\"dialog\" aria-expanded=\"true\" aria-hidden=\"true\" tabindex=\"-1\" disabled><span>{icon}{first}{icon_end}</span></button><div data-slot=\"morphing-popover-content\"{width} data-state=\"open\" role=\"dialog\" aria-modal=\"false\" aria-label=\"{aria}\" tabindex=\"-1\"><div>{}</div></div></div>",
            inner(comp, &ts, &first, None)
        ),
        Some(trigger) => {
            let trigger_id = widget_id(comp, "trigger");
            let pop_id = widget_id(comp, "pop");
            format!(
                "<div data-slot=\"morphing-popover\" data-state=\"closed\"><button type=\"button\" id=\"{trigger_id}\" data-slot=\"morphing-popover-trigger\" data-state=\"closed\" aria-haspopup=\"dialog\" aria-expanded=\"false\" popovertarget=\"{pop_id}\"><span>{icon}{trigger}{icon_end}</span></button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"morphing-popover-content\"{width} role=\"dialog\" aria-label=\"{aria}\" anchor=\"{trigger_id}\"><div>{}</div></div></div>",
                inner(comp, &ts, &trigger, Some(&pop_id))
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    const CSS: &str = include_str!("cronus_ui_css/morphing-popover.css");

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("position:absolute;z-index:20"));
        assert!(!html.contains("max-width:28rem"));
        assert!(!html.contains("popover("));
    }

    fn line(kind: &str, text: &str, cfg: &[(&str, &str)]) -> ComponentItemNode {
        ComponentItemNode {
            item_type: kind.into(),
            text: text.into(),
            link: None,
            tone: None,
            config: cfg
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    fn with_body(label: &str, body: &str) -> crate::parser::ComponentNode {
        let mut c = stub("morphing-popover", label);
        c.items.push(line("text", body, &[]));
        c
    }

    #[test]
    fn open_state_dom_matches_react() {
        let mut c = with_body("Open", "Popover body");
        c.props.insert("aria-label".into(), "Details".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<div data-slot=\"morphing-popover\" data-state=\"open\"><button type=\"button\" data-slot=\"morphing-popover-trigger\" data-state=\"open\" aria-haspopup=\"dialog\" aria-expanded=\"true\" aria-hidden=\"true\" tabindex=\"-1\" disabled><span>Open</span></button><div data-slot=\"morphing-popover-content\" data-state=\"open\" role=\"dialog\" aria-modal=\"false\" aria-label=\"Details\" tabindex=\"-1\"><div>Popover body</div></div></div>"
        );
        reject_interact(&html);
    }

    #[test]
    fn open_false_renders_closed_trigger_and_native_popover() {
        let mut c = with_body("Open", "Popover body");
        c.props.insert("aria-label".into(), "Details".into());
        c.props.insert("open".into(), "false".into());
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "pop");
        assert_eq!(
            html,
            format!(
                "<div data-slot=\"morphing-popover\" data-state=\"closed\"><button type=\"button\" id=\"{tid}\" data-slot=\"morphing-popover-trigger\" data-state=\"closed\" aria-haspopup=\"dialog\" aria-expanded=\"false\" popovertarget=\"{pid}\"><span>Open</span></button><div id=\"{pid}\" popover=\"auto\" data-slot=\"morphing-popover-content\" role=\"dialog\" aria-label=\"Details\" anchor=\"{tid}\"><div>Popover body</div></div></div>"
            )
        );
        assert!(!html.contains("disabled"));
        reject_interact(&html);
    }

    /// Docs "Feedback": glyph + label trigger, 22rem surface, sr-only label +
    /// 5-row textarea, bare footer with the native close and a disabled
    /// outline submit.
    #[test]
    fn feedback_form_with_close_and_submit() {
        let mut c = stub("morphing-popover", "Feedback");
        c.props.insert("open".into(), "false".into());
        c.props.insert("icon".into(), "message-square-plus".into());
        c.props.insert("aria-label".into(), "Send feedback".into());
        c.props.insert("width".into(), "88".into());
        c.props.insert("close".into(), "true".into());
        c.items.push(line(
            "field",
            "Your feedback",
            &[("placeholder", "Add feedback"), ("rows", "5")],
        ));
        c.items.push(line(
            "action",
            "Submit",
            &[("variant", "outline"), ("size", "sm"), ("disabled", "true")],
        ));
        let html = render(&c);
        assert!(html.contains("popovertarget=\"cui-morphing-popover-pop\"><span><svg"));
        assert!(html.contains("data-icon=\"message-square-plus\""));
        assert!(html.contains("</svg>Feedback</span></button>"));
        assert!(html.contains("data-slot=\"morphing-popover-content\" class=\"w-88\" role=\"dialog\" aria-label=\"Send feedback\""));
        assert!(html.contains("<div><form><label for=\"cui-morphing-popover-field\">Your feedback</label><textarea id=\"cui-morphing-popover-field\" rows=\"5\" placeholder=\"Add feedback\"></textarea><div data-slot=\"morphing-popover-footer\" class=\"bare\"><button type=\"button\" data-slot=\"morphing-popover-close\" aria-label=\"Close\" popovertarget=\"cui-morphing-popover-pop\" popovertargetaction=\"hide\"><svg"));
        assert!(html.contains("</button><button type=\"button\" disabled data-slot=\"button\" data-variant=\"outline\" data-size=\"sm\" class=\"cui-btn\">Submit</button></div></form></div></div></div>"));
        reject_interact(&html);
    }

    /// Docs "Quick actions": icon rows as `MorphingPopoverButton`s that hide
    /// the popover, a divider before the muted Delete row, 14rem surface.
    #[test]
    fn quick_actions_menu_rows_hide_popover() {
        let mut c = stub("morphing-popover", "Actions");
        c.props.insert("open".into(), "false".into());
        c.props.insert("icon-end".into(), "arrow-right".into());
        c.props.insert("aria-label".into(), "Quick actions".into());
        c.props.insert("width".into(), "56".into());
        c.items.push(line("item", "Edit", &[("icon", "pencil")]));
        c.items.push(line("item", "Duplicate", &[("icon", "copy")]));
        let mut delete = line(
            "item",
            "Delete",
            &[("icon", "trash-2"), ("separator", "true")],
        );
        delete.tone = Some("secondary".into());
        c.items.push(delete);
        let html = render(&c);
        assert!(html.contains("<span>Actions<svg"));
        assert!(html.contains("class=\"w-56\""));
        assert!(html.contains("<div><div data-slot=\"morphing-popover-body\" class=\"menu\"><button type=\"button\" data-slot=\"morphing-popover-button\" popovertarget=\"cui-morphing-popover-pop\" popovertargetaction=\"hide\"><svg"));
        assert!(html.contains("</svg>Edit</button><button type=\"button\" data-slot=\"morphing-popover-button\" popovertarget=\"cui-morphing-popover-pop\" popovertargetaction=\"hide\"><svg"));
        assert!(html.contains("</svg>Duplicate</button><div></div><button type=\"button\" data-slot=\"morphing-popover-button\" class=\"muted\" popovertarget=\"cui-morphing-popover-pop\" popovertargetaction=\"hide\"><svg"));
        assert!(html.ends_with("</svg>Delete</button></div></div></div></div>"));
        reject_interact(&html);
    }

    #[test]
    fn trigger_item_labels_the_closed_trigger_only() {
        let mut c = with_body("Open", "Popover body");
        c.items.push(ComponentItemNode {
            item_type: "trigger".into(),
            text: "More".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(
            html.contains("popovertarget=\"cui-morphing-popover-pop\"><span>More</span></button>")
        );
        assert!(html.contains("<div>Popover body</div>"));
        assert!(!html.contains("More</div>"));
    }

    #[test]
    fn chrome_closed_mode_hides_until_open() {
        assert!(CSS.contains(
            "[data-slot=\"morphing-popover-content\"][popover]:not(:popover-open) { display: none; }"
        ));
        assert!(CSS.contains("[data-slot=\"morphing-popover-content\"][popover]:popover-open {"));
    }

    /// The native popover sits centred on its trigger (React's absolute
    /// surface in an `inline-flex` centre) and morphs from the trigger's box
    /// on the 450 ms bounce spring; children reveal after 120 ms.
    #[test]
    fn chrome_morph_from_trigger() {
        assert!(CSS.contains(
            "top: anchor(center); inset-inline-start: anchor(center); translate: -50% -50%;"
        ));
        assert!(CSS.contains("--cui-morph-spring: 450ms linear("));
        assert!(CSS.contains("interpolate-size: allow-keywords;"));
        assert!(CSS.contains("transition: width var(--cui-morph-spring), height var(--cui-morph-spring), border-radius var(--cui-morph-spring), display 450ms allow-discrete, overlay 450ms allow-discrete;"));
        assert!(CSS.contains("@starting-style {\n  [data-slot=\"morphing-popover-content\"][popover]:popover-open { width: 8rem; height: 2.25rem; border-radius: 10px; }\n}"));
        assert!(CSS.contains("animation: cui-morphing-popover-reveal 200ms ease-out 120ms both;"));
        assert!(CSS.contains("@keyframes cui-morphing-popover-reveal {\n  from { opacity: 0; transform: translateY(6px); }\n  to { opacity: 1; transform: none; }\n}"));
        assert!(CSS.contains("[data-slot=\"morphing-popover\"]:has([popover]:popover-open) > [data-slot=\"morphing-popover-trigger\"] { pointer-events: none; }"));
        assert!(CSS.contains("[data-slot=\"morphing-popover-content\"].w-88 { min-width: 22rem; }"));
        assert!(CSS.contains("[data-slot=\"morphing-popover-content\"].w-56 { min-width: 14rem; }"));
    }

    #[test]
    fn chrome_docs_form_and_menu() {
        assert!(CSS.contains("[data-slot=\"morphing-popover-content\"] form > label {"));
        assert!(CSS.contains("[data-slot=\"morphing-popover-content\"] form > textarea {"));
        assert!(CSS.contains("padding: 1rem 1rem 0; font-size: 0.875rem; line-height: 1.25rem;"));
        assert!(CSS.contains("[data-slot=\"morphing-popover-footer\"].bare { justify-content: space-between; border-top: 0; padding: 0.25rem 0.75rem 0.75rem; }"));
        assert!(CSS.contains(
            "[data-slot=\"morphing-popover-body\"].menu { gap: 0.125rem; padding: 0.375rem; }"
        ));
        assert!(CSS.contains("[data-slot=\"morphing-popover-body\"].menu > div { margin-block: 0.25rem; height: 1px; background: var(--cronus-border); }"));
        assert!(CSS.contains("[data-slot=\"morphing-popover-button\"] {"));
        assert!(CSS.contains(
            "[data-slot=\"morphing-popover-button\"].muted { color: var(--cronus-fg-secondary); }"
        ));
        assert!(CSS.contains("[data-slot=\"morphing-popover-close\"] {"));
    }

    #[test]
    fn content_slot_is_the_contract() {
        let html = render(&stub("morphing-popover", "Open menu"));
        assert!(html.contains("data-slot=\"morphing-popover-trigger\""));
        assert!(html.contains("<span>Open menu</span></button>"));
        assert!(html.contains("aria-label=\"Details\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_surf_details() {
        let html = render(&with_body("Open menu", "First action"));
        assert!(!html.contains("<details"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&stub("morphing-popover", "Open menu"));
            reject_interact(&html);
            assert!(html.contains("data-slot=\"morphing-popover-content\""));
        });
    }

    #[test]
    fn chrome_is_token_only() {
        assert!(CSS.contains(
            "[data-slot=\"morphing-popover\"] {\n  position: relative; isolation: isolate;"
        ));
        assert!(CSS.contains("[data-slot=\"morphing-popover-trigger\"]"));
        assert!(CSS.contains(
            "[data-slot=\"morphing-popover-content\"] {\n  position: absolute; z-index: 50;"
        ));
        assert!(CSS.contains("border-radius: 16px"));
        assert!(CSS.contains("width: 18rem"));
        assert!(CSS.contains("var(--cronus-surface-floating)"));
        assert!(CSS.contains("var(--cronus-border)"));
        assert!(CSS.contains("var(--cronus-shadow-lg"));
        assert!(!CSS.contains("zinc-"));
    }
}
