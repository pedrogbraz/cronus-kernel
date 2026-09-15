//! Dedicated Select renderer — mirrors React `SelectTrigger` (Radix) closed,
//! with `SelectValue placeholder`.
//!
//! DOM: `<button data-slot="select-trigger" role="combobox"
//! aria-expanded="false" data-state="closed" data-placeholder>` > `<span>`
//! value text + lucide chevron-down.
//!
//! Zero JS: the trigger's `popovertarget` opens a native `popover="auto"`
//! `select-content` anchored under it (Esc / outside click dismiss), holding one
//! `<label data-slot="select-item" data-option="N">` per option around a
//! visually hidden radio (`name:"…"` prop, else a widget id) — so the choice
//! submits with a form and arrow keys move it. The trigger text follows the
//! checked radio in CSS: the text span carries `data-oN` labels and
//! `:has(+ select-content [data-option="N"] > :checked)` swaps in
//! `attr(data-oN)` (first 12 options). `value:"…"` pre-checks its option.
//! Gaps: picking does not close the popover (Esc / outside click / trigger do),
//! no typeahead, `aria-expanded` is not reflected, the swapped text is a CSS
//! pseudo-element (the accessible name stays `aria-label`), and the popup is a
//! `radiogroup` of native radios rather than Radix's `listbox` / `option`s.
//!
//! Text: `value:"…"` (drops `data-placeholder`), else the placeholder = label.
//! Options: choice items, else the `text` lines after the label (the audit
//! fixture repeats the placeholder as its first text; it is not an option).

use crate::cronus_ui_kit::{
    attr_nonempty, choice_texts, content_texts, esc, flag, label_of, widget_id,
};
use crate::parser::ComponentNode;

/// lucide `chevron-down` (React `[&_svg]:size-4 [&_svg]:opacity-60`).
const CHEVRON: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m6 9 6 6 6-6\"></path></svg>";

/// Options whose label the trigger can show (one CSS rule each in select.css).
const SWAP_MAX: usize = 12;

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = label_of(comp);
    let aria = attr_nonempty(comp, "aria-label")
        .map(esc)
        .unwrap_or_else(|| placeholder.clone());
    let value = attr_nonempty(comp, "value").map(esc);
    let (text, marker) = match &value {
        Some(v) => (v.clone(), ""),
        None => (placeholder.clone(), " data-placeholder=\"\""),
    };
    let options = options_of(comp, &placeholder);
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "listbox");
    let name = attr_nonempty(comp, "name")
        .map(esc)
        .unwrap_or_else(|| widget_id(comp, "value"));
    let disabled = if flag(comp, "disabled") {
        " disabled"
    } else {
        ""
    };
    let labels: String = options
        .iter()
        .take(SWAP_MAX)
        .enumerate()
        .map(|(i, o)| format!(" data-o{}=\"{o}\"", i + 1))
        .collect();
    let items: String = options
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let checked = if value.as_deref() == Some(o.as_str()) {
                " checked"
            } else {
                ""
            };
            format!(
                "<label data-slot=\"select-item\" data-option=\"{}\"><input type=\"radio\" name=\"{name}\" value=\"{o}\"{checked}><span>{o}</span></label>",
                i + 1
            )
        })
        .collect();
    format!(
        "<button type=\"button\" id=\"{trigger_id}\" role=\"combobox\" aria-expanded=\"false\" aria-autocomplete=\"none\" aria-label=\"{aria}\" data-state=\"closed\"{marker} data-slot=\"select-trigger\" popovertarget=\"{pop_id}\" aria-controls=\"{pop_id}\"{disabled}><span{labels}>{text}</span>{CHEVRON}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"select-content\" role=\"radiogroup\" aria-label=\"{aria}\" anchor=\"{trigger_id}\">{items}</div>"
    )
}

fn options_of(comp: &ComponentNode, placeholder: &str) -> Vec<String> {
    let choices = choice_texts(comp);
    let list = if choices.is_empty() {
        content_texts(comp)
    } else {
        choices
    };
    list.into_iter().filter(|o| o != placeholder).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn with_options() -> ComponentNode {
        let mut c = stub("select", "Select a plan");
        for t in ["Select a plan", "Free", "Pro"] {
            c.items.push(ComponentItemNode {
                item_type: "text".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        c
    }

    #[test]
    fn closed_trigger_matches_react_and_opens_native_radio_listbox() {
        let c = with_options();
        let html = render(&c);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "listbox");
        let name = crate::cronus_ui_kit::widget_id(&c, "value");
        assert_eq!(
            html,
            format!(
                "<button type=\"button\" id=\"{tid}\" role=\"combobox\" aria-expanded=\"false\" aria-autocomplete=\"none\" aria-label=\"Select a plan\" data-state=\"closed\" data-placeholder=\"\" data-slot=\"select-trigger\" popovertarget=\"{pid}\" aria-controls=\"{pid}\"><span data-o1=\"Free\" data-o2=\"Pro\">Select a plan</span>{CHEVRON}</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"select-content\" role=\"radiogroup\" aria-label=\"Select a plan\" anchor=\"{tid}\"><label data-slot=\"select-item\" data-option=\"1\"><input type=\"radio\" name=\"{name}\" value=\"Free\"><span>Free</span></label><label data-slot=\"select-item\" data-option=\"2\"><input type=\"radio\" name=\"{name}\" value=\"Pro\"><span>Pro</span></label></div>"
            )
        );
        assert!(!html.contains(" disabled"));
        for bad in [
            "<select",
            "<option",
            "style=",
            "v-model",
            "select-control",
            "onclick",
        ] {
            assert!(!html.contains(bad), "{bad}");
        }
    }

    #[test]
    fn value_replaces_placeholder() {
        let mut c = with_options();
        c.props.insert("value".into(), "Pro".into());
        let html = render(&c);
        assert!(html.contains("data-o2=\"Pro\">Pro</span>"));
        assert!(html.contains("value=\"Pro\" checked><span>Pro</span>"));
        assert!(!html.contains("data-placeholder"));
    }

    #[test]
    fn name_prop_disabled_and_escaping() {
        let mut c = with_options();
        c.props.insert("name".into(), "plan".into());
        c.props.insert("disabled".into(), "true".into());
        c.items[3].text = "<Pro>\"".into();
        let html = render(&c);
        assert!(html.contains("name=\"plan\" value=\"Free\">"));
        assert!(html.contains(" aria-controls=\"cui-select-listbox\" disabled><span"));
        assert!(html.contains("data-o2=\"&lt;Pro&gt;&quot;\""));
        assert!(!html.contains("<Pro>"));
    }

    #[test]
    fn chrome_listbox_popover_and_selection_swap() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"select-content\"]:popover-open {"));
        assert!(css.contains("[data-slot=\"select-item\"] {"));
        assert!(css.contains(
            "[data-slot=\"select-trigger\"]:has(+ [data-slot=\"select-content\"] [data-option=\"2\"] > :checked) > span::before { content: attr(data-o2); }"
        ));
        assert!(css.contains(
            "[data-option=\"12\"] > :checked) > span::before { content: attr(data-o12); }"
        ));
    }

    #[test]
    fn chrome_idle_look_when_disabled() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"select-trigger\"] {\n  display: flex;"));
        assert!(css.contains("[data-slot=\"select-trigger\"][data-placeholder] {"));
        assert!(!css.contains("[data-slot=\"select\"] select"));
    }
}
