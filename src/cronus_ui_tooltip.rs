//! Dedicated Tooltip renderer — mirrors React `Tooltip` with
//! `TooltipTrigger asChild` + outline `Button`.
//!
//! DOM: the trigger is `<button data-slot="button" data-variant="outline">`
//! (Radix Slot keeps the Button's own `data-slot`), followed by
//! `<div data-slot="tooltip-content" role="tooltip" popover="hint">`.
//! Zero JS: the trigger's `interestfor` (interest invokers) opens the hint
//! popover on hover / keyboard focus in browsers that support it; the trigger
//! is always `aria-describedby` the content, so the text is announced either
//! way. React's audit fixture forces `open` and portals the content out of the
//! canvas; the kernel keeps it closed (a top-layer popover cannot start open
//! without JS), so only the trigger is in the canvas on both sides.
//!
//! Content: trigger = first text (the fixture's `children` label), body =
//! the remaining texts, else the trigger text. `icon:` puts a lucide glyph in
//! the trigger; `size:icon` (or `style:tooltip+icon`) makes it the docs'
//! icon-only `Button size="icon"` whose `aria-label` is the label text (the
//! tooltip carries the same text unless a `text` item overrides it).

use crate::cronus_ui_kit::{attr_nonempty, choice, texts, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let ts = texts(comp);
    let trigger = ts.first().cloned().unwrap_or_default();
    let body = ts.iter().skip(1).cloned().collect::<Vec<_>>().join(" ");
    let body = if body.is_empty() {
        trigger.clone()
    } else {
        body
    };
    let icon = attr_nonempty(comp, "icon")
        .map(crate::cronus_ui_icons::svg_or_empty)
        .unwrap_or_default();
    let (size, inner) = match choice(comp, "size", &["icon", "icon-sm"]) {
        Some(s) => (
            format!(" data-size=\"{s}\" aria-label=\"{trigger}\""),
            icon.clone(),
        ),
        None => (String::new(), format!("{icon}{trigger}")),
    };
    let trigger_id = widget_id(comp, "tooltip-trigger");
    let content_id = widget_id(comp, "tooltip");
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"outline\"{size} interestfor=\"{content_id}\" aria-describedby=\"{content_id}\">{inner}</button><div id=\"{content_id}\" popover=\"hint\" data-slot=\"tooltip-content\" role=\"tooltip\">{body}</div>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn fixture() -> ComponentNode {
        let mut c = stub("tooltip", "Need help?");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "We usually reply within minutes.".into(),
            link: None,
            tone: None,
            config: Default::default(),
        });
        c
    }

    fn reject_js(html: &str) {
        for bad in [
            "<details", "<summary", "onclick", "onmouse", "<script", "style=", "v-data",
        ] {
            assert!(!html.contains(bad), "{bad} in {html}");
        }
    }

    #[test]
    fn trigger_is_outline_button_and_content_is_hint_popover() {
        assert_eq!(
            render(&fixture()),
            "<button type=\"button\" id=\"cui-tooltip-tooltip-trigger\" data-slot=\"button\" data-variant=\"outline\" interestfor=\"cui-tooltip-tooltip\" aria-describedby=\"cui-tooltip-tooltip\">Need help?</button><div id=\"cui-tooltip-tooltip\" popover=\"hint\" data-slot=\"tooltip-content\" role=\"tooltip\">We usually reply within minutes.</div>"
        );
        reject_js(&render(&fixture()));
    }

    #[test]
    fn label_only_uses_label_as_body() {
        let html = render(&stub("tooltip", "Hint"));
        assert!(html.contains(">Hint</button>"));
        assert!(html.contains("role=\"tooltip\">Hint</div>"));
        assert!(!html.contains("data-slot=\"tooltip\""));
    }

    #[test]
    fn icon_only_trigger_and_icon_label_trigger() {
        let mut c = stub("tooltip", "Add to library");
        c.props.insert("icon".into(), "plus".into());
        c.props.insert("size".into(), "icon".into());
        let html = render(&c);
        assert!(html.starts_with("<button type=\"button\" id=\"cui-tooltip-tooltip-trigger\" data-slot=\"button\" data-variant=\"outline\" data-size=\"icon\" aria-label=\"Add to library\" interestfor=\"cui-tooltip-tooltip\" aria-describedby=\"cui-tooltip-tooltip\"><svg "), "{html}");
        assert!(html.contains("data-icon=\"plus\""));
        assert!(html.contains("</svg></button><div id=\"cui-tooltip-tooltip\" popover=\"hint\" data-slot=\"tooltip-content\" role=\"tooltip\">Add to library</div>"));
        let mut h = fixture();
        h.props.insert("icon".into(), "help-circle".into());
        let html = render(&h);
        assert!(html.contains("data-icon=\"help-circle\""));
        assert!(html.contains("</svg>Need help?</button>"));
        assert!(!html.contains("data-size"));
        reject_js(&html);
    }

    #[test]
    fn chrome_closed_until_popover_open() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("animation: cronus-pop-in 150ms var(--ease-out-quart) both;"));
        assert!(
            css.contains("[data-slot=\"tooltip-content\"]:not(:popover-open) { display: none; }")
        );
        assert!(css.contains("[data-slot=\"tooltip-content\"]:popover-open {"));
        assert!(!css.contains("[data-slot=\"tooltip-trigger\"]"));
    }
}
