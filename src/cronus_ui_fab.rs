//! Dedicated Fab renderer. DOM matches React:
//! `<div data-slot="fab"><button type="button" aria-label><span><svg/></span></button></div>`.
//! The main button has no `data-slot` in React, so CSS targets
//! `[data-slot="fab"] > button`. The label is the accessible name only — never
//! visible text; `icon:"plus"` picks the lucide glyph (default `plus`).
//!
//! Speed dial (`action "Write a note" icon:pencil` items): zero JS. The main
//! button sits in a `<label>` after a visually hidden checkbox that carries
//! the open state (click, Space, focus ring); the `<ul data-slot="fab-actions">`
//! (each `<li>` = label chip + round action button, React order) follows and
//! CSS shows it while checked, staggering the items in with the snappy spring
//! from the bottom up and rotating the main glyph 45°, like the motion
//! variants. The decorated button is `aria-hidden` / `tabindex="-1"`;
//! `aria-expanded` is not reflected (no JS).
//! Not interact `buttonish()` (`data-slot="button"` + inline primary styles).

use crate::cronus_ui_kit::{attr_nonempty, esc, item_icon, label_of};
use crate::parser::ComponentNode;

const PLUS_ICON: &str = "<svg viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" aria-hidden=\"true\"><path d=\"M12 5v14M5 12h14\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let label = label_of(comp);
    let icon = attr_nonempty(comp, "icon")
        .and_then(crate::cronus_ui_icons::svg)
        .unwrap_or_else(|| PLUS_ICON.to_string());
    let actions: Vec<String> = comp
        .items
        .iter()
        .filter(|i| i.item_type == "action" && !i.text.is_empty())
        .map(|i| {
            let t = esc(&i.text);
            let glyph = item_icon(i);
            format!(
                "<li><span>{t}</span><button type=\"button\" aria-label=\"{t}\">{glyph}</button></li>"
            )
        })
        .collect();
    if actions.is_empty() {
        return format!(
            "<div data-slot=\"fab\"><button type=\"button\" aria-label=\"{label}\"><span>{icon}</span></button></div>"
        );
    }
    let id = crate::cronus_ui_kit::instance_id(comp, "fab");
    format!(
        "<div data-slot=\"fab\" data-speed-dial=\"\"><label><input type=\"checkbox\" id=\"{id}\" aria-label=\"{label}\"><button type=\"button\" aria-label=\"{label}\" tabindex=\"-1\" aria-hidden=\"true\"><span>{icon}</span></button></label><ul data-slot=\"fab-actions\">{}</ul></div>",
        actions.join("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;

    fn reject_interact(html: &str) {
        assert!(!html.contains("data-slot=\"button\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("onclick="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("animate"));
        assert!(!html.contains("<input"));
        assert!(!html.contains("<label"));
    }

    #[test]
    fn root_is_wrapper_with_icon_button_no_slot_no_text() {
        // Wave 1t: React's main button has no data-slot and no visible text.
        let html = render(&stub("fab", "Create"));
        assert_eq!(
            html,
            format!("<div data-slot=\"fab\"><button type=\"button\" aria-label=\"Create\"><span>{PLUS_ICON}</span></button></div>")
        );
        assert!(!html.contains("fab-button"));
        assert!(!html.contains(">Create<"));
        reject_interact(&html);
    }

    #[test]
    fn label_is_aria_label() {
        let html = render(&stub("fab", "New item"));
        assert!(html.contains("aria-label=\"New item\""));
        assert!(!html.contains("aria-expanded"));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_buttonish() {
        let c = stub("fab", "Compose");
        let html = render(&c);
        assert!(!html.contains("data-slot=\"button\""));
        reject_interact(&html);
    }

    #[test]
    fn actions_render_speed_dial_after_checkbox() {
        let mut c = stub("fab", "Create");
        c.items.push(crate::parser::ComponentItemNode {
            item_type: "action".into(),
            text: "Write a note".into(),
            link: None,
            tone: None,
            config: [("icon".to_string(), "pencil".to_string())]
                .into_iter()
                .collect(),
        });
        let html = render(&c);
        assert!(html.contains("<input type=\"checkbox\""));
        assert!(html.contains("<ul data-slot=\"fab-actions\"><li><span>Write a note</span><button type=\"button\" aria-label=\"Write a note\"><svg"));
        assert!(html.contains("data-icon=\"pencil\""));
        assert!(!html.contains("style="));
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"fab\"]"));
        assert!(
            css.contains("[data-slot=\"fab\"] > button, [data-slot=\"fab\"] > label > button {")
        );
        assert!(
            css.contains("[data-slot=\"fab\"] > button svg, [data-slot=\"fab\"] > label > button svg {\n  width: 1.5rem; height: 1.5rem;")
        );
        assert!(!css.contains("[data-slot=\"fab-button\"]"));
        assert!(css.contains("display: inline-flex"));
        assert!(css.contains("width: 3.5rem; height: 3.5rem"));
        assert!(css.contains("border-radius: 9999px"));
        assert!(css.contains("var(--cronus-primary)"));
        assert!(css.contains("var(--cronus-primary-foreground)"));
        assert!(!css.contains("zinc-"));
    }
}
