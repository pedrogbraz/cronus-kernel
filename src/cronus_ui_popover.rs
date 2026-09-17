//! Dedicated Popover renderer. DOM mirrors React (`PopoverTrigger asChild` +
//! `Button`): the trigger is `<button data-slot="button" data-variant="primary">`
//! (Radix Slot keeps the Button's own `data-slot`), followed by a native
//! `popover="auto"` `<div data-slot="popover-content" role="dialog">`.
//! Zero JS: the content is closed until the trigger's `popovertarget` opens it
//! (React's audit fixture forces `defaultOpen` and portals the open content out
//! of the canvas; the kernel cannot open a top-layer popover without JS).
//! Trigger: the label, styled by `trigger-variant:` (React Button variants,
//! `primary` by default). Docs content: a `title "…"` heading (`h4`) with the
//! `description:"…"` paragraph, then `field "Width" value:"100%"` rows
//! (`cronus_ui_dialog::fields_html`) in the docs' three-column grid with
//! `h-8` inputs; without those, the extra `text` items are the body.
//! `width:80` (Tailwind `w-80`) and `align:start|end` (Radix `align`) have no
//! React attribute, so they are `w-80` / `align-start` classes on the content.
//! Not interact `popover("popover")` SURF `<details>` overlay.

use crate::cronus_ui_dialog::fields_html;
use crate::cronus_ui_kit::{attr, attr_nonempty, choice, esc, item, label_of, texts, widget_id};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let ts = texts(comp);
    let trigger = ts.first().cloned().unwrap_or_else(|| label_of(comp));
    let aria = aria_label_of(comp).unwrap_or_else(|| "Details".into());
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "pop");
    let variant = match attr(comp, "trigger-variant").map(str::trim) {
        Some("outline") => "outline",
        Some("secondary") => "secondary",
        Some("ghost") => "ghost",
        Some("link") => "link",
        Some("destructive") | Some("danger") => "destructive",
        _ => "primary",
    };
    let heading = item(comp, "title").filter(|t| !t.is_empty()).map(esc);
    let description = attr_nonempty(comp, "description").map(esc);
    let fields = fields_html(comp, &pop_id);
    let body = if heading.is_some() || description.is_some() || !fields.is_empty() {
        let head = match (heading, description) {
            (None, None) => String::new(),
            (h, d) => format!(
                "<div class=\"head\">{}{}</div>",
                h.map(|h| format!("<h4>{h}</h4>")).unwrap_or_default(),
                d.map(|d| format!("<p>{d}</p>")).unwrap_or_default()
            ),
        };
        let grid = if fields.is_empty() {
            String::new()
        } else {
            format!("<div class=\"grid\">{fields}</div>")
        };
        format!("<div class=\"stack\">{head}{grid}</div>")
    } else {
        let body = ts.iter().skip(1).cloned().collect::<Vec<_>>().join("");
        if body.is_empty() {
            trigger.clone()
        } else {
            body
        }
    };
    let mut classes: Vec<String> = Vec::new();
    if let Some(w) = attr_nonempty(comp, "width").map(str::trim) {
        if matches!(w, "64" | "72" | "80" | "96") {
            classes.push(format!("w-{w}"));
        }
    }
    if let Some(a) = choice(comp, "align", &["start", "end"]) {
        classes.push(format!("align-{a}"));
    }
    let class = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" data-slot=\"button\" data-variant=\"{variant}\" popovertarget=\"{pop_id}\" aria-haspopup=\"dialog\">{trigger}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"popover-content\" role=\"dialog\" aria-label=\"{aria}\" anchor=\"{trigger_id}\"{class}>{body}</div>"
    )
}

fn aria_label_of(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(|v| esc(v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn reject_interact(html: &str) {
        assert!(!html.contains("<details"));
        assert!(!html.contains("<summary"));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("data-slot=\"popover\">"));
    }

    fn with_body(label: &str, body: &str) -> crate::parser::ComponentNode {
        let mut c = stub("popover", label);
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: body.into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        c
    }

    #[test]
    fn trigger_is_primary_button_slot_like_react_as_child() {
        let mut c = with_body("Open", "Popover body");
        c.props.insert("aria-label".into(), "Details".into());
        let html = render(&c);
        assert_eq!(
            html,
            "<button type=\"button\" id=\"cui-popover-trigger\" data-slot=\"button\" data-variant=\"primary\" popovertarget=\"cui-popover-pop\" aria-haspopup=\"dialog\">Open</button><div id=\"cui-popover-pop\" popover=\"auto\" data-slot=\"popover-content\" role=\"dialog\" aria-label=\"Details\" anchor=\"cui-popover-trigger\">Popover body</div>"
        );
        assert!(!html.contains("data-slot=\"popover-trigger\""));
        reject_interact(&html);
    }

    #[test]
    fn content_slot_is_the_contract() {
        let html = render(&stub("popover", "More"));
        assert!(html.contains("data-slot=\"popover-content\""));
        assert!(html.contains(">More</button>"));
        assert!(html.contains("aria-label=\"Details\""));
        assert!(!html.contains("data-slot=\"popover\">"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_surf_details() {
        let html = render(&with_body("More", "Extra actions."));
        assert!(!html.contains("<details"));
        reject_interact(&html);
    }

    #[test]
    fn docs_dimensions_popover_has_heading_and_field_grid() {
        let mut c = stub("popover", "Dimensions");
        c.props.insert("trigger-variant".into(), "outline".into());
        c.props.insert("width".into(), "80".into());
        c.props.insert("align".into(), "start".into());
        c.props.insert(
            "description".into(),
            "Set the dimensions for the layer.".into(),
        );
        c.items.push(ComponentItemNode {
            item_type: "title".into(),
            text: "Dimensions".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let mut w = ComponentItemNode {
            item_type: "field".into(),
            text: "Width".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        };
        w.config.insert("value".into(), "100%".into());
        c.items.push(w);
        let html = render(&c);
        assert!(html.starts_with("<button type=\"button\" id=\"cui-popover-trigger\" data-slot=\"button\" data-variant=\"outline\" popovertarget=\"cui-popover-pop\" aria-haspopup=\"dialog\">Dimensions</button><div id=\"cui-popover-pop\" popover=\"auto\" data-slot=\"popover-content\" role=\"dialog\" aria-label=\"Details\" anchor=\"cui-popover-trigger\" class=\"w-80 align-start\"><div class=\"stack\"><div class=\"head\"><h4>Dimensions</h4><p>Set the dimensions for the layer.</p></div><div class=\"grid\"><div class=\"field\"><label data-slot=\"label\" for=\"cui-popover-pop-f1\">Width</label><input data-slot=\"input\" id=\"cui-popover-pop-f1\" type=\"text\" value=\"100%\" /></div></div></div></div>"), "{html}");
        reject_interact(&html);
        let css = crate::cronus_ui::component_chrome_css();
        assert!(
            css.contains("[data-slot=\"popover-content\"].w-80:popover-open { min-width: 20rem; }")
        );
        assert!(css.contains("[data-slot=\"popover-content\"] > .stack > .grid > .field {\n  display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); align-items: center; gap: 0.75rem;"));
        assert!(css.contains("[data-slot=\"popover-content\"] > .stack > .grid > .field > [data-slot=\"input\"] { grid-column: span 2 / span 2; height: 2rem; }"));
        assert!(css.contains("[data-slot=\"popover-content\"]:popover-open {\n  display: block; z-index: 50; width: 18rem;"));
        assert!(css.contains("animation: cronus-pop-in 180ms var(--ease-out-quart) both;"));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"popover-content\"]:popover-open {\n  display: block; z-index: 50; width: 18rem;"));
        assert!(css.contains("[data-slot=\"button\"]:has(+ [data-slot=\"popover-content\"])"));
        assert!(css.contains("padding: 0.75rem"));
        assert!(css.contains("var(--cronus-surface-floating)"));
        assert!(css.contains("var(--cronus-border)"));
        assert!(css.contains("var(--cronus-shadow-lg"));
        assert!(!css.contains("button:has(+ [data-slot=\"popover-content\"])"));
        assert!(!css.contains("zinc-"));
    }
}
