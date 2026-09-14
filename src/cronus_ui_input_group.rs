//! Dedicated InputGroup renderer. DOM matches React:
//! `<div data-slot="input-group"><div data-slot="input-group-addon" data-align="start">…</div><input data-slot="input" />`.
//! Not interact `input("input-group")` (`<label data-slot="input-group"><input data-slot="input-group-control">`).

use crate::cronus_ui_kit::{attr_nonempty, esc, item};
use crate::parser::ComponentNode;

pub fn render(comp: &ComponentNode) -> String {
    let placeholder = placeholder_of(comp);
    let mut input = String::from("<input data-slot=\"input\" type=\"text\"");
    if !placeholder.is_empty() {
        input.push_str(&format!(" placeholder=\"{placeholder}\""));
    }
    if let Some(label) = aria_label_of(comp) {
        input.push_str(&format!(" aria-label=\"{label}\""));
    }
    input.push_str(" />");
    match addon_of(comp) {
        Some(addon) => format!(
            "<div data-slot=\"input-group\"><div data-slot=\"input-group-addon\" data-align=\"start\">{addon}</div>{input}</div>"
        ),
        None => format!("<div data-slot=\"input-group\">{input}</div>"),
    }
}

fn aria_label_of(comp: &ComponentNode) -> Option<String> {
    attr_nonempty(comp, "aria-label").map(esc)
}

fn placeholder_of(comp: &ComponentNode) -> String {
    placeholder_raw(comp).map(esc).unwrap_or_default()
}

fn placeholder_raw(comp: &ComponentNode) -> Option<&str> {
    comp.props
        .get("placeholder")
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .or_else(|| item(comp, "placeholder").filter(|s| !s.is_empty()))
        .or_else(|| item(comp, "text").filter(|s| !s.is_empty()))
}

fn addon_of(comp: &ComponentNode) -> Option<String> {
    if let Some(v) = comp.props.get("addon").filter(|s| !s.is_empty()) {
        return Some(esc(v));
    }
    if let Some(v) = item(comp, "addon").filter(|s| !s.is_empty()) {
        return Some(esc(v));
    }
    let ph = placeholder_raw(comp);
    let mut primary: Option<&str> = None;
    let mut extra: Option<&str> = None;
    for i in &comp.items {
        if i.text.is_empty() {
            continue;
        }
        if ph == Some(i.text.as_str()) && matches!(i.item_type.as_str(), "text" | "placeholder") {
            continue;
        }
        if primary.is_none() {
            primary = Some(i.text.as_str());
        } else if extra.is_none() {
            extra = Some(i.text.as_str());
        }
    }
    extra.or(primary).map(esc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cronus_ui_kit::stub;
    use crate::parser::ComponentItemNode;
    use std::collections::HashMap;

    fn reject_interact(html: &str) {
        assert!(!html.contains("<label"));
        assert!(!html.contains("data-slot=\"input-group-control\""));
        assert!(!html.contains("style="));
        assert!(!html.contains("v-data="));
        assert!(!html.contains("v-model="));
    }

    #[test]
    fn root_is_div_with_input_slot_not_label_control() {
        let html = render(&stub("input-group", "https://"));
        assert!(html.starts_with("<div "));
        assert!(html.contains("data-slot=\"input-group\""));
        assert!(
            html.contains("data-slot=\"input-group-addon\" data-align=\"start\">https://</div>")
        );
        assert!(html.contains("<input data-slot=\"input\" type=\"text\""));
        reject_interact(&html);
        assert_eq!(
            html,
            "<div data-slot=\"input-group\"><div data-slot=\"input-group-addon\" data-align=\"start\">https://</div><input data-slot=\"input\" type=\"text\" /></div>"
        );
    }

    #[test]
    fn wave1t_default_fixture_exact_dom() {
        // app.cronus InputGroupDefault: label "Amount", text "0.00", text "$" + aria-label:"Amount".
        let mut c = stub("input-group", "Amount");
        for t in ["0.00", "$"] {
            c.items.push(ComponentItemNode {
                item_type: "text".into(),
                text: t.into(),
                link: None,
                tone: None,
                config: HashMap::new(),
            });
        }
        let last = c.items.len() - 1;
        c.items[last]
            .config
            .insert("aria-label".into(), "Amount".into());
        assert_eq!(
            render(&c),
            "<div data-slot=\"input-group\"><div data-slot=\"input-group-addon\" data-align=\"start\">$</div><input data-slot=\"input\" type=\"text\" placeholder=\"0.00\" aria-label=\"Amount\" /></div>"
        );
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains(
            "font-size: 0.875rem; line-height: 1.25rem;\n}\n[data-slot=\"input-group-addon\"]"
        ));
    }

    #[test]
    fn addon_from_props() {
        let mut c = stub("input-group", "Amount");
        c.props.insert("addon".into(), "$".into());
        let html = render(&c);
        assert!(html.contains("data-slot=\"input-group-addon\" data-align=\"start\">$</div>"));
        reject_interact(&html);
    }

    #[test]
    fn addon_from_first_extra_item() {
        let mut c = stub("input-group", "URL");
        c.items.push(ComponentItemNode {
            item_type: "item".into(),
            text: "https://".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(
            html.contains("data-slot=\"input-group-addon\" data-align=\"start\">https://</div>")
        );
        reject_interact(&html);
    }

    #[test]
    fn placeholder_from_text_item() {
        let mut c = stub("input-group", "https://");
        c.items.push(ComponentItemNode {
            item_type: "text".into(),
            text: "example.com".into(),
            link: None,
            tone: None,
            config: HashMap::new(),
        });
        let html = render(&c);
        assert!(
            html.contains("data-slot=\"input-group-addon\" data-align=\"start\">https://</div>")
        );
        assert!(html.contains("placeholder=\"example.com\""));
        reject_interact(&html);
    }

    #[test]
    fn placeholder_from_props() {
        let mut c = stub("input-group", "@");
        c.props.insert("placeholder".into(), "username".into());
        let html = render(&c);
        assert!(html.contains("data-slot=\"input-group-addon\" data-align=\"start\">@</div>"));
        assert!(html.contains("placeholder=\"username\""));
        reject_interact(&html);
    }

    #[test]
    fn skips_interact_label_control() {
        let html = render(&stub("input-group", "https://"));
        assert!(!html.contains("input-group-control"));
        reject_interact(&html);
    }

    #[test]
    fn chrome_is_token_only() {
        let css = crate::cronus_ui::component_chrome_css();
        assert!(css.contains("[data-slot=\"input-group\"]"));
        assert!(css.contains("[data-slot=\"input-group-addon\"]"));
        assert!(css.contains("display: flex"));
        assert!(css.contains("height: 2.5rem"));
        assert!(css.contains("align-items: stretch"));
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("var(--cronus-surface-inset)"));
        assert!(css.contains("var(--cronus-fg-tertiary)"));
        assert!(!css.contains("zinc-"));
    }
}
