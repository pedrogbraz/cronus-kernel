//! Dedicated Combobox renderer. DOM matches React's closed Combobox: a single
//! outline `<button data-slot="combobox-trigger" role="combobox">` with a
//! `<span>` label (placeholder or selected option) + lucide `chevrons-up-down`.
//! React has no wrapper element and mounts the listbox only while open.
//! Opening, filtering and picking need JS, so the trigger is React's native
//! button rendered `disabled` with React's idle look (not dimmed); a real
//! `disabled` prop adds `data-disabled` and dims like React `disabled:opacity-50`.
//! Not interact `select("combobox")` (`<label><select data-slot="combobox-control">`).

use crate::cronus_ui_kit::{choice_texts, esc, item};
use crate::parser::ComponentNode;

/// lucide `chevrons-up-down` (React `size-4 opacity-60 ml-2`).
const CHEVRONS: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"m7 15 5 5 5-5\"></path><path d=\"m7 9 5-5 5 5\"></path></svg>";

pub fn render(comp: &ComponentNode) -> String {
    let options = choice_texts(comp);
    let selected = selected_of(comp, &options);
    let trigger = selected.clone().unwrap_or_else(|| placeholder_of(comp));
    let mut attrs = String::from(
        "type=\"button\" data-slot=\"combobox-trigger\" data-variant=\"outline\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\"",
    );
    if let Some(aria) = comp.props.get("aria-label").filter(|s| !s.is_empty()) {
        attrs.push_str(&format!(" aria-label=\"{}\"", esc(aria)));
    }
    attrs.push_str(" data-state=\"closed\"");
    if selected.is_none() {
        attrs.push_str(" data-placeholder=\"\"");
    }
    if disabled(comp) {
        attrs.push_str(" data-disabled=\"\"");
    }
    attrs.push_str(" disabled");
    format!("<button {attrs}><span>{trigger}</span>{CHEVRONS}</button>")
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
        assert!(!html.contains("<label"));
        assert!(!html.contains(" style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onclick="));
    }

    #[test]
    fn root_is_single_disabled_trigger_button() {
        let html = render(&combo("Search", &["Ada", "Grace"]));
        assert!(
            !html.contains("data-slot=\"combobox\""),
            "React has no wrapper slot: {html}"
        );
        assert!(!html.contains("popover"), "no JS-less listbox: {html}");
        reject_interact(&html);
        assert_eq!(
            html,
            format!("<button type=\"button\" data-slot=\"combobox-trigger\" data-variant=\"outline\" role=\"combobox\" aria-expanded=\"false\" aria-haspopup=\"listbox\" data-state=\"closed\" data-placeholder=\"\" disabled><span>Search</span>{CHEVRONS}</button>")
        );
    }

    #[test]
    fn label_is_placeholder_not_an_option() {
        let html = render(&combo("Search", &["Ada", "Grace"]));
        assert!(html.contains("<span>Search</span>"));
        assert!(
            !html.contains("Ada"),
            "options only mount while open: {html}"
        );
        reject_interact(&html);
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
        assert!(html.contains("<span>Grace</span>"));
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
        assert!(html.contains(" data-disabled=\"\" disabled><span>Search</span>"));
        reject_interact(&html);
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
