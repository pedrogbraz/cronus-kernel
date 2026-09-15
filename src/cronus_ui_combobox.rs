//! Dedicated Combobox renderer. DOM matches React's closed Combobox: a single
//! outline `<button data-slot="combobox-trigger" role="combobox">` with a
//! `<span>` label (placeholder or selected option) + lucide `chevrons-up-down`.
//! React has no wrapper element and mounts the listbox (`combobox-content`)
//! only while open.
//! Zero JS: the trigger's `popovertarget` opens a native `popover="auto"`
//! `combobox-content` anchored under it (Esc / outside click dismiss) with one
//! `<label data-slot="combobox-item" data-option="N">` per option around a
//! visually hidden radio (`name:"…"` prop, else a widget id): the choice submits
//! with a form. The trigger text follows the checked radio in CSS (`data-oN`
//! labels on the text span + `attr()`, first 12 options), like select.
//! Gaps: no search input / filtering (cmdk needs JS), picking does not close the
//! popover, `aria-expanded` is not reflected, the swapped text is a CSS
//! pseudo-element, and the popup is a native `radiogroup`, not a listbox.
//! A real `disabled` prop adds `data-disabled`, `disabled` and dims like React
//! `disabled:opacity-50`.
//! Not interact `select("combobox")` (`<label><select data-slot="combobox-control">`).

use crate::cronus_ui_kit::{choice_texts, content_texts, esc, item, widget_id};
use crate::parser::ComponentNode;

/// lucide `chevrons-up-down` (React `size-4 opacity-60 ml-2`).
const CHEVRONS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m7 15 5 5 5-5\"></path><path d=\"m7 9 5-5 5 5\"></path></svg>";

/// Options whose label the trigger can show (one CSS rule each in combobox.css).
const SWAP_MAX: usize = 12;

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = placeholder_of(comp);
    let options = options_of(comp, &placeholder);
    let selected = selected_of(comp, &options);
    let trigger = selected.clone().unwrap_or_else(|| placeholder.clone());
    let trigger_id = widget_id(comp, "trigger");
    let pop_id = widget_id(comp, "listbox");
    let aria_prop = comp
        .props
        .get("aria-label")
        .filter(|s| !s.is_empty())
        .map(|a| esc(a));
    let mut attrs = format!(
        "type=\"button\" id=\"{trigger_id}\" data-slot=\"combobox-trigger\" data-variant=\"outline\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\""
    );
    if let Some(aria) = &aria_prop {
        attrs.push_str(&format!(" aria-label=\"{aria}\""));
    }
    attrs.push_str(" data-state=\"closed\"");
    if selected.is_none() {
        attrs.push_str(" data-placeholder=\"\"");
    }
    if disabled(comp) {
        attrs.push_str(" data-disabled=\"\"");
    }
    attrs.push_str(&format!(
        " popovertarget=\"{pop_id}\" aria-controls=\"{pop_id}\""
    ));
    if disabled(comp) {
        attrs.push_str(" disabled");
    }
    let name = comp
        .props
        .get("name")
        .filter(|s| !s.is_empty())
        .map(|n| esc(n))
        .unwrap_or_else(|| widget_id(comp, "value"));
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
            let checked = if selected.as_deref() == Some(o.as_str()) {
                " checked"
            } else {
                ""
            };
            format!(
                "<label data-slot=\"combobox-item\" data-option=\"{}\"><input type=\"radio\" name=\"{name}\" value=\"{o}\"{checked}><span>{o}</span></label>",
                i + 1
            )
        })
        .collect();
    let aria = aria_prop.unwrap_or_else(|| placeholder.clone());
    format!(
        "<button {attrs}><span{labels}>{trigger}</span>{CHEVRONS}</button><div id=\"{pop_id}\" popover=\"auto\" data-slot=\"combobox-content\" role=\"radiogroup\" aria-label=\"{aria}\" anchor=\"{trigger_id}\">{items}</div>"
    )
}

/// Choice items, else the `text` lines after the label, minus the placeholder
/// (the audit fixture repeats it as its first text).
fn options_of(comp: &ComponentNode, placeholder: &str) -> Vec<String> {
    let choices = choice_texts(comp);
    let list = if choices.is_empty() {
        content_texts(comp)
    } else {
        choices
    };
    list.into_iter().filter(|o| o != placeholder).collect()
}

fn placeholder_of(comp: &ComponentNode) -> String {
    if let Some(v) = comp.props.get("placeholder").filter(|s| !s.is_empty()) {
        return esc(v);
    }
    if let Some(t) = item(comp, "placeholder").filter(|s| !s.is_empty()) {
        return esc(t);
    }
    for kind in ["label", "title"] {
        if let Some(t) = item(comp, kind).filter(|s| !s.is_empty()) {
            return esc(t);
        }
    }
    "Select an option…".into()
}

fn selected_of(comp: &ComponentNode, options: &[String]) -> Option<String> {
    if let Some(v) = comp.props.get("value").filter(|s| !s.is_empty()) {
        let e = esc(v);
        if options.iter().any(|o| o == &e) {
            return Some(e);
        }
    }
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        if is_true(i.config.get("selected")) || is_true(i.config.get("checked")) {
            let e = esc(&i.text);
            if options.iter().any(|o| o == &e) {
                return Some(e);
            }
        }
    }
    None
}

fn disabled(comp: &ComponentNode) -> bool {
    comp.props
        .get("disabled")
        .map(|s| s == "true")
        .unwrap_or(false)
}

fn is_true(raw: Option<&String>) -> bool {
    matches!(raw.map(String::as_str), Some("true" | "on" | "1"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;

    fn extra(text: &str) -> ComponentItemNode {
        ComponentItemNode {
            item_type: "item".into(),
            text: text.into(),
            link: None,
            tone: None,
            config: Default::default(),
        }
    }

    fn combo(placeholder: &str, items: &[&str]) -> crate::parser::ComponentNode {
        let mut c = stub("combobox", placeholder);
        for t in items {
            c.items.push(extra(t));
        }
        c
    }

    fn reject_interact(html: &str) {
        assert!(!html.contains("<select"));
        assert!(!html.contains("</select>"));
        assert!(!html.contains("data-slot=\"combobox-control\""));
        assert!(!html.contains(" style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn trigger_opens_native_popover_radio_listbox() {
        let c = combo("Search", &["Ada", "Grace"]);
        let html = render(&c);
        assert!(
            !html.contains("data-slot=\"combobox\""),
            "React has no wrapper slot: {html}"
        );
        reject_interact(&html);
        let tid = crate::cronus_ui_kit::widget_id(&c, "trigger");
        let pid = crate::cronus_ui_kit::widget_id(&c, "listbox");
        let name = crate::cronus_ui_kit::widget_id(&c, "value");
        assert_eq!(
            html,
            format!("<button type=\"button\" id=\"{tid}\" data-slot=\"combobox-trigger\" data-variant=\"outline\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\" data-state=\"closed\" data-placeholder=\"\" popovertarget=\"{pid}\" aria-controls=\"{pid}\"><span data-o1=\"Ada\" data-o2=\"Grace\">Search</span>{CHEVRONS}</button><div id=\"{pid}\" popover=\"auto\" data-slot=\"combobox-content\" role=\"radiogroup\" aria-label=\"Search\" anchor=\"{tid}\"><label data-slot=\"combobox-item\" data-option=\"1\"><input type=\"radio\" name=\"{name}\" value=\"Ada\"><span>Ada</span></label><label data-slot=\"combobox-item\" data-option=\"2\"><input type=\"radio\" name=\"{name}\" value=\"Grace\"><span>Grace</span></label></div>")
        );
    }

    #[test]
    fn label_is_placeholder_not_an_option() {
        let html = render(&combo("Search", &["Ada", "Grace"]));
        assert!(html.contains(">Search</span>"));
        assert!(!html.contains("value=\"Search\""), "{html}");
        reject_interact(&html);
    }

    /// Emitted fixture: `text` lines after the label are the options.
    #[test]
    fn fixture_text_lines_are_options_without_the_placeholder() {
        let mut c = stub("combobox", "Select fruit");
        for t in ["Select fruit", "Apple", "Banana"] {
            c.items.push(ComponentItemNode {
                item_type: "text".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: Default::default(),
            });
        }
        let html = render(&c);
        assert!(html.contains("data-o1=\"Apple\" data-o2=\"Banana\">Select fruit</span>"));
        assert_eq!(html.matches("data-slot=\"combobox-item\"").count(), 2);
    }

    #[test]
    fn aria_label_from_props() {
        let mut c = combo("Select fruit", &["Apple", "Banana"]);
        c.props.insert("aria-label".into(), "Fruit".into());
        let html = render(&c);
        assert!(html.contains(" aria-label=\"Fruit\" data-state=\"closed\""));
    }

    #[test]
    fn value_selects_matching_option() {
        let mut c = combo("Search", &["Ada", "Grace"]);
        c.props.insert("value".into(), "Grace".into());
        let html = render(&c);
        assert!(html.contains("data-o2=\"Grace\">Grace</span>"));
        assert!(html.contains("value=\"Grace\" checked><span>Grace</span>"));
        assert!(!html.contains("data-placeholder"));
        reject_interact(&html);
    }

    #[test]
    fn selected_item_config_marks_option() {
        let mut c = combo("Search", &["Ada", "Grace"]);
        c.items[1].config.insert("selected".into(), "true".into());
        let html = render(&c);
        assert!(html.contains("<span>Ada</span>"));
        reject_interact(&html);
    }

    #[test]
    fn disabled_prop_marks_data_disabled() {
        let mut c = combo("Search", &["Ada"]);
        c.props.insert("disabled".into(), "true".into());
        let html = render(&c);
        assert!(html.contains(" data-disabled=\"\" popovertarget="));
        assert!(html.contains(" disabled><span data-o1=\"Ada\">Search</span>"));
        reject_interact(&html);
        let open = render(&combo("Search", &["Ada"]));
        assert!(
            !open.contains(" disabled"),
            "only a real disabled prop disables"
        );
    }

    #[test]
    fn chrome_listbox_popover_and_selection_swap() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"combobox-content\"]:popover-open {"));
        assert!(css.contains("[data-slot=\"combobox-item\"] {"));
        assert!(css.contains(
            "[data-slot=\"combobox-trigger\"]:has(+ [data-slot=\"combobox-content\"] [data-option=\"1\"] > :checked) > span::before { content: attr(data-o1); }"
        ));
    }

    #[test]
    fn skips_interact_native_select() {
        let c = combo("Search", &["Ada", "Grace"]);
        let html = render(&c);
        assert!(!html.contains("combobox-control"));
        assert!(!html.contains("<select"));
        reject_interact(&html);
    }

    #[test]
    fn no_voodoo_even_when_runtime_on() {
        crate::voodoo::with_enabled(true, || {
            let html = render(&combo("Search", &["Ada", "Grace"]));
            reject_interact(&html);
        });
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"combobox-trigger\"]"));
        assert!(css.contains("justify-content: space-between"));
        assert!(!css.contains("zinc-"));
    }

    /// Wave 1t geometry: React outline Button `h-10 px-4 w-full justify-between
    /// font-normal`, transparent background, text-sm/20px, placeholder in
    /// fg-tertiary; chevron is an svg (the old `::after` caret is gone); the
    /// JS-only `disabled` trigger keeps the idle look, only `data-disabled` dims.
    #[test]
    fn chrome_matches_react_geometry() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "[data-slot=\"combobox-trigger\"] {\n  display: inline-flex; align-items: center; justify-content: space-between; gap: 0.5rem;\n  width: 100%; height: 2.5rem; padding: 0 1rem; box-sizing: border-box; white-space: nowrap;\n  border-radius: var(--cronus-radius-lg); border: 1px solid var(--cronus-border);\n  background: transparent; color: var(--cronus-fg); box-shadow: var(--cronus-shadow-xs, none);\n  font-family: inherit; font-size: 0.875rem; line-height: 1.25rem; font-weight: 400; cursor: default;\n}"
        ));
        assert!(css.contains("[data-slot=\"combobox-trigger\"][data-placeholder] { color: var(--cronus-fg-tertiary); }"));
        assert!(css.contains("[data-slot=\"combobox-trigger\"][data-disabled] { opacity: 0.5; }"));
        assert!(!css.contains("[data-slot=\"combobox-trigger\"]:disabled"));
        assert!(!css.contains("[data-slot=\"combobox-trigger\"]::after"));
    }
}
